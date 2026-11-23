
int add(int a, int b)
{
    return a + b;
}

int main()
{

    int a = 0;
    for (int i = 0; i < 100000000; i++)
    {
        a = add(a, i);
    }

    return a;
}